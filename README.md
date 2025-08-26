# rust-svelte-setup

## Connect to database

```bash
docker exec -it ea94d01bbc5d psql -U postgres -d user_db
```

## Load Test
```
oha -z 30s -c 50 -p 30 http://localhost:3000/user/me -H 'Cookie:session_token=DYBnxArmbc8hTpXVlxXOpKhH.4vsZnDBMaztg1AKFzAFBHPsv'
```
