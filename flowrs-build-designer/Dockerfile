FROM node:alpine

WORKDIR /app/

COPY package.json ./
COPY yarn.lock ./
COPY nuxt.config.ts ./

RUN yarn cache clean
RUN yarn install --frozen-lockfile

COPY . ./app/

CMD ["yarn", "nodemon"]
