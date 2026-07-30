===description===
Two overlapping `psr-0` entries map to the same logical relative path
(`Legacy/Thing.php`) from different base directories: a catch-all empty-
prefix entry (`""`, matches any class name — a legacy full-tree autoload
some Zend1/Magento1 projects use) and a specific `Legacy_` prefix. Composer's
own resolution order tries longest-prefix-first, so the specific entry must
win. Each candidate file declares a differently-named method so a false
resolution is caught immediately as UndefinedMethod instead of silently
passing.
===file:composer.json===
{"autoload":{"psr-0":{"":"vendor/legacy_all/","Legacy_":"vendor/legacy_specific/"}}}
===file:vendor/legacy_all/Legacy/Thing.php===
<?php
class Legacy_Thing {
    public function fromCatchall(): void {}
}
===file:vendor/legacy_specific/Legacy/Thing.php===
<?php
class Legacy_Thing {
    public function fromSpecific(): void {}
}
===file:Consumer.php===
<?php
$t = new Legacy_Thing();
$t->fromSpecific();
===expect===
