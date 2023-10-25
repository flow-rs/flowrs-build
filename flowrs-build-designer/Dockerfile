FROM node:20-alpine

WORKDIR /app

COPY package.json ./
COPY yarn.lock ./
# COPY tsconfig.json ./
COPY nuxt.config.ts ./
# COPY 
RUN yarn install --frozen-lockfile

COPY . /app

CMD ["yarn", "nodemon"]
