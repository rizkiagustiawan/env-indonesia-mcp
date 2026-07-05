# Production Deployment Guide

## Using Docker
1. Copy `.env.example` to `.env` and fill in your API keys.
2. Build the Docker image: `docker build -t env-indonesia-mcp .`
3. Run the Docker container: `docker run -d --name env-indonesia-mcp --env-file .env env-indonesia-mcp`

## Using systemd
1. Copy `.env.example` to `.env` and fill in your API keys.
2. Copy `env-indonesia-mcp.service` to `/etc/systemd/system/`.
3. Reload systemd: `sudo systemctl daemon-reload`
4. Start the service: `sudo systemctl start env-indonesia-mcp`
5. Enable on boot: `sudo systemctl enable env-indonesia-mcp`
