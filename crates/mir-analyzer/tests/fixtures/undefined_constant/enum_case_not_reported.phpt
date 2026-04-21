===source===
<?php
enum Status {
    case Active;
    case Inactive;
}
function test(): Status {
    return Status::Active;
}
===expect===
