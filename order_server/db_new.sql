drop table if exists orders;
create table orders (
       id serial primary key,

       items_id varchar(140) not null,
       price varchar(140) not null,
       total_price INT not null,

       currency varchar(2000),

       sub_time TIMESTAMP default now(),
       pay_time TIMESTAMP default '1970-01-01 00:00:00',

       description varchar(30)
);


-- insert into orders (items_id, price, total_price, currency, sub_time, pay_time, description)
-- values(       '',
--        '',
--        10000,
--        'RMB',
--        '2023-03-10 10:00:00',
--        '1970-01-01 00:00:00',
--        '');
