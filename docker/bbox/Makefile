up:
	docker-compose -f docker-compose-bbox.yml -f docker-compose-postgresql.yml -f docker-compose-nginx.yml -f docker-compose-metrics.yml -f docker-compose-auth.yml -f docker-compose-windmill.yml up -d

ps:
	docker-compose -f docker-compose-bbox.yml -f docker-compose-postgresql.yml -f docker-compose-nginx.yml -f docker-compose-auth.yml -f docker-compose-windmill.yml -f docker-compose-metrics.yml ps

logs:
	docker-compose -f docker-compose-bbox.yml -f docker-compose-postgresql.yml -f docker-compose-nginx.yml -f docker-compose-auth.yml -f docker-compose-windmill.yml -f docker-compose-metrics.yml logs -f --tail=20

down:
	docker-compose -f docker-compose-bbox.yml -f docker-compose-postgresql.yml -f docker-compose-nginx.yml -f docker-compose-auth.yml -f docker-compose-windmill.yml -f docker-compose-metrics.yml down

resetdb:
	docker stop bbox_postgresql_1
	docker rm bbox_postgresql_1
	docker volume rm bbox_postgres-data	
