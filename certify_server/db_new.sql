drop table if exists users;
create table users (
       id UUID DEFAULT gen_random_uuid() PRIMARY KEY,

       email varchar(200),
       password_hash varchar(200),

       create_time TIMESTAMP default now()
);

-- 创建索引
-- CREATE INDEX idx_user_email ON user (email);
