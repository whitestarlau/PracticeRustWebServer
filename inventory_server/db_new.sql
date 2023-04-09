drop table if exists inventory;
create table inventory (
       id serial primary key,
       
       count INT not null,

       description varchar(140)
);

insert into inventory (count , description) values(10000,'test_goods');


drop table if exists inventory_change;
create table inventory_change (
       id serial primary key,
       
       count INT not null,

       inventory_id INT not null,
       
       deduction_order_id INT,

       description varchar(140)
);

insert into inventory_change (count ,inventory_id, deduction_order_id, description) values(1,1, null ,'test_goods');
