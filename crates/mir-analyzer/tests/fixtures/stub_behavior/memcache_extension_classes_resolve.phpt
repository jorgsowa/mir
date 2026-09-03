===description===
FP-I1: the `memcache` PECL extension (Memcache, MemcachePool) had no
vendored stubs/ dir despite PhpStormStubsMap.php already listing every
entry — same missing-stub root cause as the fixed C6 (ast).
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php

function connect(string $host): Memcache {
    $m = new Memcache();
    $m->connect($host, 11211);
    return $m;
}

function usePool(MemcachePool $pool): void {
    $pool->close();
}
===expect===
