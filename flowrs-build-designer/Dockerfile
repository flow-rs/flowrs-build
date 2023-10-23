FROM node:20

WORKDIR /

COPY . .

RUN yarn install
RUN yarn build

WORKDIR /.output
RUN ls -al
CMD [ "node", "./server/index.mjs"]
