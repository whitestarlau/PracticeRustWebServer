-- 扣减库存消息表，本来需要创建在本地
drop table if exists orders_de_inventory_msg;
create table orders_de_inventory_msg (
       id serial primary key,

       user_id UUID not null,

       order_id INT not null,

       description varchar(140)
);
