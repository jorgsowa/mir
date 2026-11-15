===description===
PHP forbids adding a type to an inherited untyped property, so a redeclaration is not flagged; a private parent property is not inherited and still is.
===file===
<?php
class Command {
    protected $signature;
//  ^^^^^^^^^^^^^^^^^^^^ MissingPropertyType: Property Command::$signature has no type annotation
    private $secret;
//  ^^^^^^^^^^^^^^^ MissingPropertyType: Property Command::$secret has no type annotation
}

class BaseCommand extends Command {}

class CheckLinks extends BaseCommand {
    protected $signature = 'links:check';

    private $secret = 'x';
//  ^^^^^^^^^^^^^^^^^^^^^ MissingPropertyType: Property CheckLinks::$secret has no type annotation

    public $own;
//  ^^^^^^^^^^^ MissingPropertyType: Property CheckLinks::$own has no type annotation
}
