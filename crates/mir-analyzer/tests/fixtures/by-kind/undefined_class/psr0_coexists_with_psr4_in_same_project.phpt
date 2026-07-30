===description===
A project migrating off PSR-0 gradually (the Magento 1 / ZF1 pattern) declares
both a `psr-4` section (new code) and a `psr-0` section (legacy code) in the
same `composer.json`. Both must lazily resolve from the same `Psr4Map`
without one shadowing the other, and each class's genuinely-undefined method
call must still surface (proving both were actually loaded, not silently
degraded to `mixed`).
===file:composer.json===
{"autoload":{"psr-4":{"App\\":"src/"},"psr-0":{"Legacy_":"legacy/"}}}
===file:src/Service.php===
<?php
namespace App;
class Service {
    public function run(): string { return 'ok'; }
}
===file:legacy/Legacy/Helper.php===
<?php
class Legacy_Helper {
    public function assist(): string { return 'ok'; }
}
===file:Consumer.php===
<?php
use App\Service;
class Consumer {
    public function handle(): void {
        $s = new Service();
        $s->nope();
        $h = new Legacy_Helper();
        $h->nope();
    }
}
===expect===
Consumer.php: UndefinedMethod@6:8-6:18: Method App\Service::nope() does not exist
Consumer.php: UndefinedMethod@8:8-8:18: Method Legacy_Helper::nope() does not exist
