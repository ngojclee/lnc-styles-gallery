# Stage 1: Build React + Vite + TanStack Router SPA (ui/dist)
FROM node:20-alpine AS ui-builder

WORKDIR /ui
COPY ui/package.json ui/pnpm-lock.yaml* ./
RUN corepack enable && pnpm install --no-frozen-lockfile
COPY ui/ ./
RUN pnpm build

# Stage 2: FastAPI + Streamable HTTP MCP + Static SPA Runtime
FROM python:3.11-slim

ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    STYLES_HOST=0.0.0.0 \
    STYLES_PORT=8000 \
    STYLES_DATA_ROOT=/data

WORKDIR /app

COPY requirements.txt /app/requirements.txt
RUN pip install --no-cache-dir --upgrade pip && \
    pip install --no-cache-dir -r /app/requirements.txt

COPY VERSION /app/VERSION
COPY app /app/app
COPY ui /app/ui
COPY --from=ui-builder /ui/dist /app/ui/dist

# Default empty data dir (overridden by volume mount /data in docker-compose)
RUN mkdir -p /data/refero-styles /data/prompt-driven-ui

EXPOSE 8000

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD python -c "import urllib.request; urllib.request.urlopen('http://127.0.0.1:8000/health', timeout=3)" || exit 1

CMD ["uvicorn", "app.main:app", "--host", "0.0.0.0", "--port", "8000"]
