===description===
DuplicateEnum fires for a namespaced enum declared twice in the same file.
===file===
<?php
namespace App;

enum Status {
    case Active;
}

enum Status {
    case Inactive;
}
===expect===
DuplicateEnum@8:0-10:1: Enum App\Status has already been defined
