===description===
FALSE POSITIVE reproducer. @inheritdoc resolution should walk the full ancestor
chain, so a grandchild with @inheritdoc picks up the grandparent's @return type
when the intermediate class also has @inheritdoc (and no explicit return type).
===config===
suppress=UnusedVariable,UnusedParam
php_version=8.2
===file===
<?php
class Product {}

abstract class AbstractStore {
    /** @return Product */
    abstract public function get(int $id): mixed;
}

abstract class CachedStore extends AbstractStore {
    /** @inheritdoc */
    public function get(int $id): mixed {
        return new Product();
    }
}

class SqlStore extends CachedStore {
    /** @inheritdoc */
    public function get(int $id): mixed {
        return new Product();
    }
}

function test(SqlStore $store): void {
    $p = $store->get(1);
    /** @mir-check $p is Product */
    echo get_class($p);
}
===expect===
