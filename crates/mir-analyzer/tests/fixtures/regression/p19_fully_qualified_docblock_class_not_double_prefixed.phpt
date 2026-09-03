===description===
FP-P19 control: a fully-qualified docblock class name (leading `\`) must still
be used as-is, not prefixed a second time with the current namespace — guards
against the `resolve_type_name` fix over-correcting the qualified-but-relative
case into also re-qualifying already-absolute names.
===config===
suppress=UnusedParam,MissingPropertyType
===file:Warning.php===
<?php
namespace App\Warning;

class Warning {}
===file:Container.php===
<?php
namespace App;

class Container {
    /** @var \App\Warning\Warning */
    private $warning;
}
===expect===
