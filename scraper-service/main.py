"""
Market Intelligence Scraper Service
Microserviço de scraping e análise de tendências de mercado
"""
from fastapi import FastAPI, HTTPException, BackgroundTasks, Depends
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, Field
from typing import Optional, List
from datetime import datetime
from enum import Enum
import asyncio
import redis.asyncio as redis
from sqlalchemy.ext.asyncio import AsyncSession, create_async_engine
from sqlalchemy.orm import sessionmaker
import os
from dotenv import load_dotenv

load_dotenv()

# ============================================================================
# CONFIGURAÇÃO
# ============================================================================

app = FastAPI(
    title="Arcsat Market Intelligence API",
    description="Scraping e análise de tendências de marketplace",
    version="1.0.0"
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # Configure adequadamente em produção
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Database
DATABASE_URL = os.getenv("DATABASE_URL", "").replace("postgresql://", "postgresql+asyncpg://")
engine = create_async_engine(DATABASE_URL, echo=True)
AsyncSessionLocal = sessionmaker(engine, class_=AsyncSession, expire_on_commit=False)

# Redis
redis_client = None

# ============================================================================
# MODELS
# ============================================================================

class MarketplaceEnum(str, Enum):
    AMAZON = "amazon"
    MERCADO_LIVRE = "mercado_livre"
    SHOPEE = "shopee"
    B2W = "b2w"
    MAGALU = "magalu"

class ScrapingStatus(str, Enum):
    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"

class ScrapingJobCreate(BaseModel):
    marketplace: MarketplaceEnum
    search_query: str
    category: Optional[str] = None
    max_pages: int = Field(default=5, ge=1, le=50)
    priority: int = Field(default=5, ge=1, le=10)

class ScrapingJobResponse(BaseModel):
    job_id: str
    status: ScrapingStatus
    marketplace: MarketplaceEnum
    search_query: str
    created_at: datetime
    message: str

class ProductTrend(BaseModel):
    product_id: str
    marketplace: MarketplaceEnum
    title: str
    price: float
    sales_rank: Optional[int] = None
    rating: Optional[float] = None
    num_reviews: int = 0
    seller_name: str
    listing_date: Optional[datetime] = None
    scraped_at: datetime

class TrendAnalysis(BaseModel):
    category: str
    marketplace: MarketplaceEnum
    avg_price: float
    median_price: float
    top_sellers: List[str]
    trending_keywords: List[str]
    growth_rate: float
    competition_level: str  # low, medium, high
    analyzed_at: datetime

# ============================================================================
# DEPENDENCIES
# ============================================================================

async def get_db():
    async with AsyncSessionLocal() as session:
        yield session

async def get_redis():
    return redis_client

# ============================================================================
# SCRAPING CORE
# ============================================================================

class ScraperEngine:
    """Engine principal de scraping com Playwright"""

    def __init__(self):
        self.browser = None
        self.context = None

    async def initialize(self):
        """Inicializa o navegador com configurações stealth"""
        from playwright.async_api import async_playwright

        self.playwright = await async_playwright().start()

        proxy_config = None
        if os.getenv("PROXY_ENABLED", "false").lower() == "true":
            proxy_config = {
                "server": f"http://{os.getenv('PROXY_HOST')}:{os.getenv('PROXY_PORT')}",
                "username": os.getenv("PROXY_USERNAME"),
                "password": os.getenv("PROXY_PASSWORD")
            }

        self.browser = await self.playwright.chromium.launch(
            headless=True,
            args=[
                '--disable-blink-features=AutomationControlled',
                '--disable-web-security',
                '--disable-features=IsolateOrigins,site-per-process'
            ],
            proxy=proxy_config
        )

    async def create_stealth_context(self):
        """Cria contexto de navegação com fingerprinting aleatório"""
        import random

        viewports = [
            {"width": 1920, "height": 1080},
            {"width": 1366, "height": 768},
            {"width": 1536, "height": 864},
        ]

        user_agents = [
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        ]

        context = await self.browser.new_context(
            viewport=random.choice(viewports),
            user_agent=random.choice(user_agents),
            locale="pt-BR",
            timezone_id="America/Sao_Paulo"
        )

        # Injetar scripts anti-detecção
        await context.add_init_script("""
            Object.defineProperty(navigator, 'webdriver', { get: () => undefined });
            window.chrome = { runtime: {} };
            Object.defineProperty(navigator, 'plugins', { get: () => [1, 2, 3, 4, 5] });
        """)

        return context

    async def scrape_amazon(self, query: str, max_pages: int = 5):
        """Scraper específico para Amazon"""
        context = await self.create_stealth_context()
        page = await context.new_page()

        results = []

        try:
            # Navegar para busca
            await page.goto(f"https://www.amazon.com.br/s?k={query.replace(' ', '+')}")
            await asyncio.sleep(random.uniform(2, 4))

            for page_num in range(max_pages):
                # Extrair produtos da página
                products = await page.query_selector_all('[data-component-type="s-search-result"]')

                for product in products:
                    try:
                        title_elem = await product.query_selector('h2 a span')
                        title = await title_elem.inner_text() if title_elem else "N/A"

                        price_elem = await product.query_selector('.a-price-whole')
                        price_text = await price_elem.inner_text() if price_elem else "0"
                        price = float(price_text.replace('.', '').replace(',', '.'))

                        rating_elem = await product.query_selector('.a-icon-alt')
                        rating_text = await rating_elem.inner_text() if rating_elem else "0"
                        rating = float(rating_text.split()[0].replace(',', '.'))

                        results.append({
                            "title": title,
                            "price": price,
                            "rating": rating,
                            "marketplace": "amazon",
                            "scraped_at": datetime.utcnow()
                        })
                    except Exception as e:
                        print(f"Erro ao processar produto: {e}")
                        continue

                # Próxima página
                next_button = await page.query_selector('.s-pagination-next')
                if next_button and page_num < max_pages - 1:
                    await next_button.click()
                    await asyncio.sleep(random.uniform(3, 6))
                else:
                    break

        finally:
            await context.close()

        return results

    async def scrape_mercado_livre(self, query: str, max_pages: int = 5):
        """Scraper específico para Mercado Livre"""
        context = await self.create_stealth_context()
        page = await context.new_page()

        results = []

        try:
            await page.goto(f"https://lista.mercadolivre.com.br/{query.replace(' ', '-')}")
            await asyncio.sleep(random.uniform(2, 4))

            for page_num in range(max_pages):
                products = await page.query_selector_all('.ui-search-layout__item')

                for product in products:
                    try:
                        title_elem = await product.query_selector('.ui-search-item__title')
                        title = await title_elem.inner_text() if title_elem else "N/A"

                        price_elem = await product.query_selector('.andes-money-amount__fraction')
                        price_text = await price_elem.inner_text() if price_elem else "0"
                        price = float(price_text.replace('.', '').replace(',', '.'))

                        results.append({
                            "title": title,
                            "price": price,
                            "marketplace": "mercado_livre",
                            "scraped_at": datetime.utcnow()
                        })
                    except Exception as e:
                        continue

                # Próxima página
                next_button = await page.query_selector('.andes-pagination__button--next')
                if next_button and page_num < max_pages - 1:
                    await next_button.click()
                    await asyncio.sleep(random.uniform(3, 6))
                else:
                    break

        finally:
            await context.close()

        return results

    async def close(self):
        if self.browser:
            await self.browser.close()
        if self.playwright:
            await self.playwright.stop()

# Instância global
scraper_engine = ScraperEngine()

# ============================================================================
# API ENDPOINTS
# ============================================================================

@app.on_event("startup")
async def startup():
    """Inicialização do serviço"""
    global redis_client
    redis_client = await redis.from_url(os.getenv("REDIS_URL", "redis://localhost:6379"))
    await scraper_engine.initialize()
    print("🚀 Scraper Service iniciado")

@app.on_event("shutdown")
async def shutdown():
    """Limpeza ao desligar"""
    if redis_client:
        await redis_client.close()
    await scraper_engine.close()

@app.get("/")
async def root():
    return {"service": "Arcsat Market Intelligence", "status": "running"}

@app.get("/health")
async def health_check():
    return {"status": "healthy", "timestamp": datetime.utcnow()}

@app.post("/api/v1/scraping/jobs", response_model=ScrapingJobResponse)
async def create_scraping_job(
    job: ScrapingJobCreate,
    background_tasks: BackgroundTasks,
    redis_conn = Depends(get_redis)
):
    """
    Cria um job de scraping e adiciona à fila
    """
    import uuid

    job_id = str(uuid.uuid4())

    # Salvar job no Redis
    job_data = {
        "job_id": job_id,
        "status": ScrapingStatus.PENDING,
        "marketplace": job.marketplace,
        "search_query": job.search_query,
        "category": job.category,
        "max_pages": job.max_pages,
        "created_at": datetime.utcnow().isoformat()
    }

    await redis_conn.hset(f"scraping:job:{job_id}", mapping=job_data)
    await redis_conn.lpush(f"scraping:queue:{job.priority}", job_id)

    # Adicionar tarefa em background
    background_tasks.add_task(process_scraping_job, job_id, job)

    return ScrapingJobResponse(
        job_id=job_id,
        status=ScrapingStatus.PENDING,
        marketplace=job.marketplace,
        search_query=job.search_query,
        created_at=datetime.utcnow(),
        message="Job criado e adicionado à fila"
    )

@app.get("/api/v1/scraping/jobs/{job_id}")
async def get_job_status(job_id: str, redis_conn = Depends(get_redis)):
    """
    Consulta o status de um job de scraping
    """
    job_data = await redis_conn.hgetall(f"scraping:job:{job_id}")

    if not job_data:
        raise HTTPException(status_code=404, detail="Job não encontrado")

    return job_data

@app.get("/api/v1/trends/{marketplace}")
async def get_trends(
    marketplace: MarketplaceEnum,
    category: Optional[str] = None,
    limit: int = 100
):
    """
    Retorna produtos em alta e análise de tendências
    """
    # TODO: Implementar consulta ao banco de dados
    # Por enquanto retorna mock
    return {
        "marketplace": marketplace,
        "category": category,
        "trends": [],
        "message": "Implementar consulta ao banco"
    }

# ============================================================================
# WORKER FUNCTIONS
# ============================================================================

async def process_scraping_job(job_id: str, job: ScrapingJobCreate):
    """
    Processa um job de scraping em background
    """
    try:
        # Atualizar status para running
        await redis_client.hset(f"scraping:job:{job_id}", "status", ScrapingStatus.RUNNING)

        # Executar scraping baseado no marketplace
        if job.marketplace == MarketplaceEnum.AMAZON:
            results = await scraper_engine.scrape_amazon(job.search_query, job.max_pages)
        elif job.marketplace == MarketplaceEnum.MERCADO_LIVRE:
            results = await scraper_engine.scrape_mercado_livre(job.search_query, job.max_pages)
        else:
            raise ValueError(f"Marketplace {job.marketplace} não suportado")

        # Salvar resultados no Redis (cache temporário)
        await redis_client.set(
            f"scraping:results:{job_id}",
            str(results),  # JSON seria melhor
            ex=3600  # 1 hora
        )

        # TODO: Salvar no PostgreSQL

        # Atualizar status para completed
        await redis_client.hset(f"scraping:job:{job_id}", "status", ScrapingStatus.COMPLETED)
        await redis_client.hset(f"scraping:job:{job_id}", "results_count", len(results))

    except Exception as e:
        await redis_client.hset(f"scraping:job:{job_id}", "status", ScrapingStatus.FAILED)
        await redis_client.hset(f"scraping:job:{job_id}", "error", str(e))
        print(f"Erro no job {job_id}: {e}")

# ============================================================================
# ANÁLISE DE TENDÊNCIAS
# ============================================================================

@app.post("/api/v1/analyze/trends")
async def analyze_trends(marketplace: MarketplaceEnum, category: str):
    """
    Analisa tendências baseado em dados coletados
    TODO: Implementar lógica de análise com pandas/numpy
    """
    return {
        "message": "Análise de tendências - A implementar",
        "marketplace": marketplace,
        "category": category
    }

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(
        "main:app",
        host=os.getenv("API_HOST", "0.0.0.0"),
        port=int(os.getenv("API_PORT", 8001)),
        reload=True
    )
