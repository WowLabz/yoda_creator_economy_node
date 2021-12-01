FROM paritytech/ci-linux:production

WORKDIR /var/www/yoda_creator_economy_node
COPY . /var/www/yoda_creator_economy_node
EXPOSE 9944