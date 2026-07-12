===description===
UndefinedDocblockClass does NOT fire when a `@psalm-import-type ... from`
docblock tag's source class exists.
===file===
<?php
/** @psalm-type UserId = int */
class UserRepository {}

/** @psalm-import-type UserId from UserRepository */
class A {}
===expect===
