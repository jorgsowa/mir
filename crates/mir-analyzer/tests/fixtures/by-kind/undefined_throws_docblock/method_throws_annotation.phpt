===description===
UndefinedThrowsDocblock fires when a method's `@throws` docblock names a
class that does not exist.
===file===
<?php
class Service {
    /**
     * @throws NonExistentServiceException
     */
    public function run(): void {
//                  ^^^ UndefinedThrowsDocblock: @throws class 'NonExistentServiceException' does not exist
    }
}
