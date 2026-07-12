===description===
A class named only in a `@psalm-import-type ... from` docblock tag must
not be reported UnusedClass.
===config===
suppress=
===file===
<?php
/** @psalm-type UserId = int */
final class OnlyUsedViaImportTypeFrom {}

/** @psalm-import-type UserId from OnlyUsedViaImportTypeFrom */
class Consumer {}
===expect===
