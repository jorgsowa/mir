===description===
FALSE POSITIVE reproducer. @inheritdoc on a trait method should inherit the
interface's @return type so callers receive the correct type.
===config===
suppress=UnusedVariable,UnusedParam
php_version=8.2
===file===
<?php
class Entity {}

interface EntityFinder {
    /** @return Entity */
    public function findOne(int $id): mixed;
}

trait FinderTrait {
    /** @inheritdoc */
    public function findOne(int $id): mixed {
        return new Entity();
    }
}

class Repository implements EntityFinder {
    use FinderTrait;
}

function test(Repository $repo): void {
    $e = $repo->findOne(1);
    /** @mir-check $e is Entity */
    echo get_class($e);
}
===expect===
