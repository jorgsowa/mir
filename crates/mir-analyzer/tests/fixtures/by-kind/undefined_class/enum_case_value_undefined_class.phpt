===description===
An enum case's value expression referencing an undefined class must report
UndefinedClass — the enum-analysis loop previously matched only methods, so
case values were never walked at all.
===file===
<?php
enum Status: string {
    case Active = UndefinedClass::VALUE;
}
===expect===
UndefinedClass@3:18-3:32: Class UndefinedClass does not exist
