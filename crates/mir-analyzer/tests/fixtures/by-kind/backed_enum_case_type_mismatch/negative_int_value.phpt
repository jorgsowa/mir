===description===
P6(a): Negative integer values are valid for int-backed enums and must not be flagged.
===file===
<?php
enum Offset: int {
    case Before = -1;
    case None = 0;
    case After = 1;
}
===expect===
