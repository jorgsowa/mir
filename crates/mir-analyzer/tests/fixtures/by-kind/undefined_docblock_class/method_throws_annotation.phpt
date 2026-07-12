===description===
UndefinedDocblockClass fires when a method's `@throws` docblock names a
class that does not exist.
===file===
<?php
class Service {
    /**
     * @throws NonExistentServiceException
     */
    public function run(): void {
    }
}
===expect===
UndefinedDocblockClass@6:20-6:23: Docblock type 'NonExistentServiceException' does not exist
