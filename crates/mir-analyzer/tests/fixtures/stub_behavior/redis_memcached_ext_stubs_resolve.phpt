===description===
Bundled redis/memcached extension stubs resolve, including a top-level class imported via `use` into a namespaced file (the phpredis/Memcached usage pattern in Laravel) — both class-constant fetches and instantiation

===config===
suppress=UnusedVariable,UnusedParam

===file===
<?php

namespace Illuminate\Redis\Connections;

use Redis;
use Memcached;

class PacksValues
{
    public function redisConstViaImport(): mixed
    {
        // `use Redis;` imports the top-level \Redis; resolving this class
        // constant must not report `Redis` as an undefined class.
        return Redis::OPT_SERIALIZER;
    }

    public function memcachedConstViaImport(): mixed
    {
        return Memcached::OPT_BINARY_PROTOCOL;
    }

    public function newRedisViaImport(): Redis
    {
        return new Redis();
    }

    public function newRedisFullyQualified(): \Redis
    {
        return new \Redis();
    }

    public function newMemcachedViaImport(): Memcached
    {
        return new Memcached();
    }
}
?>
===expect===
