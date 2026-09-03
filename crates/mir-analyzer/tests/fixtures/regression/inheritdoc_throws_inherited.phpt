===description===
FALSE POSITIVE reproducer. @inheritdoc should inherit @throws from the parent
so the child method body is not flagged for throwing a declared exception.
===config===
php_version=8.2
===file===
<?php
class NotFoundException extends \RuntimeException {}

interface Finder {
    /**
     * @return string
     * @throws NotFoundException
     */
    public function find(int $id): mixed;
}

class DbFinder implements Finder {
    /** @inheritdoc */
    public function find(int $id): mixed {
        if ($id === 0) {
            throw new NotFoundException('not found');
        }
        return 'result';
    }
}
===expect===
