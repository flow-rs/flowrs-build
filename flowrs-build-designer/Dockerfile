FROM node:alpine

WORKDIR /app/

COPY package.json ./
COPY yarn.lock ./
COPY nuxt.config.ts ./

RUN yarn cache clean
RUN yarn install
RUN yarn add @pinia/nuxt
RUN yarn add json-editor-vue

COPY . ./app/

CMD ["yarn", "nodemon"]
