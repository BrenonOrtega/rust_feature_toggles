CREATE TABLE feature_toggles (
    name varchar(100) primary key not null,
    state tinyint not null DEFAULT ,
);