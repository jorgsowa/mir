===description===
Symfony rejects a host containing `/`, so `getHost()` and `getHttpHost()` can't choose a path, but stay tainted for other sinks.
===file===
<?php
namespace Symfony\Component\HttpFoundation {
    class Request {
        public function getHost(): string { return ''; }
        public function getHttpHost(): string { return ''; }
    }
}

namespace {
    function boot(Symfony\Component\HttpFoundation\Request $r): void {
        require __DIR__ . '/sites/' . $r->getHttpHost() . '/settings.php';
        require __DIR__ . '/sites/' . $r->getHost() . '/settings.php';
        echo $r->getHost();
//      ^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    }
}
