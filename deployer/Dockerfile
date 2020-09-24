FROM node:12.16.1-buster-slim

RUN apt-get update \
    && apt-get install -y bash net-tools curl vim \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/* \
    && curl -L "https://github.com/docker/compose/releases/download/1.23.1/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose \
    && chmod +x /usr/local/bin/docker-compose


RUN mkdir /app /nginx /controller
WORKDIR /app
#COPY ./package*.json ./
COPY . .

RUN npm install --production


EXPOSE 3002
CMD ["npm", "run", "start"]
