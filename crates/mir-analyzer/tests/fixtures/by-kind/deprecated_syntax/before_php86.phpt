===description===
Before PHP 8.6 none of its compile-time deprecations are reported.
===config===
<mir>
  <phpVersion>8.5</phpVersion>
</mir>
===file===
<?php
class Let {
    const namespace = 'app';

    public function __construct() {
        return null;
    }

    public function __debugInfo(): ?array {
        return [];
    }
}

function load(callable $read): mixed {
    try {
        return $read();
    } finally {
        return null;
    }
}
