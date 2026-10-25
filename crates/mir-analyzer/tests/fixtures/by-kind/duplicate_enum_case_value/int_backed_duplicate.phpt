===description===
Two int cases with the same value are a PHP fatal; the later case is reported, also for another spelling of the same number.
===file===
<?php
enum Status: int {
    case Active = 1;
    case Enabled = 1;
//                 ^ DuplicateEnumCaseValue: Duplicate value in enum Status for cases Active and Enabled
    case Hex = 0x1;
//             ^^^ DuplicateEnumCaseValue: Duplicate value in enum Status for cases Active and Hex
    case Inactive = 2;
}
