===description===
DuplicateEnum fires for a namespaced enum declared twice in the same file.
===file===
<?php
namespace App;

enum Status {
    case Active;
}

enum Status {
//<^ +2:1 DuplicateEnum: Enum App\Status has already been defined
    case Inactive;
}
