FROM oven/bun AS base
WORKDIR /usr/src/app

FROM base AS install
RUN mkdir -p /temp/dev
COPY package.json bun.lockb /temp/dev/
RUN cd /temp/dev && bun install --frozen-lockfile

RUN mkdir -p /temp/prod
COPY package.json bun.lockb /temp/prod/
RUN cd /temp/prod && bun install --frozen-lockfile --production

FROM base AS prerelease
COPY --from=install /temp/prod/node_modules node_modules
COPY . .

RUN bun test

FROM base AS release
COPY --from=prerelease . .

EXPOSE 6565/tcp
ENTRYPOINT [  "bun", "run", "/usr/src/app/index.ts", "--database", "/var/lib/tiles/tiles.db", "--port", "6565" ]