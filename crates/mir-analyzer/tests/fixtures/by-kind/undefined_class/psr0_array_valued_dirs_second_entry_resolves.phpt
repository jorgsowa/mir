===description===
A single `psr-0` prefix mapped to an array of base directories (Magento 1's
`app/code/{core,community,local}` split under one autoload root is the
real-world shape) — the target class exists only in the second directory, so
`resolve()` must keep trying candidates after the first miss instead of
stopping at the first directory in the list.
===file:composer.json===
{"autoload":{"psr-0":{"Legacy_":["vendor/legacy/first/","vendor/legacy/second/"]}}}
===file:vendor/legacy/second/Legacy/Thing.php===
<?php
class Legacy_Thing {
    public function name(): string { return 'thing'; }
}
===file:Consumer.php===
<?php
$t = new Legacy_Thing();
echo $t->name();
$t->nope();
===expect===
Consumer.php: UndefinedMethod@4:0-4:10: Method Legacy_Thing::nope() does not exist
