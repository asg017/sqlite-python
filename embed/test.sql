.bail on




.load ./dist/params

select get_x();

select set_x(100);

select get_x();

.exit



.load ./usaddress0

select usaddresses.*
from json_each('[
    "123 Main St. Suite 100 Chicago, IL",
    "11604 Whittier Blvd, Whittier, CA 90601",
    "5005 Paramount Blvd, Pico Rivera, CA 90660",
    "5600 Whittier Blvd, Commerce, CA 90022"
  ]') as addresses
join usaddresses(addresses.value);


.exit

.load ./scalar

select scalar_noparams();

select scalar_param1('hi');

select scalar_paramoptional(), scalar_paramoptional(1);

select py_value(scalar_param_multiple_optional(1));
select py_value(scalar_param_multiple_optional(1, 2));
select py_value(scalar_param_multiple_optional(1, 2, 3));
.exit

.load ./a
select a();

.load ./b
select b();

.load ./pysimple0

.header on
.mode box

select simple_version(), simple_reverse("abc xyz");

select * from pragma_table_xinfo('simple_product');

select * from simple_product('abc', 'xyz');

select * from simple_all();


.load ./py_pdfplumber0

select pdfplumber_version();
select pdfplumber_debug();