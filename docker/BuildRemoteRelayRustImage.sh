docker stop RemoteRelay
docker rm RemoteRelay
docker build -t remoterelay:latest . -f docker/Dockerfile

# docker run --restart=unless-stopped --add-host=host.docker.internal:host-gateway -d --name RemoteRelay -p 8010:8000 remoterelay:latest
docker run --restart=unless-stopped -d --name RemoteRelay -p 5000:5000 remoterelay:latest
