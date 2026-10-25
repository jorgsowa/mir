===description===
A pure enum cannot implement BackedEnum, directly or through another interface.
===file===
<?php
interface HasValue extends BackedEnum {}

enum Direct implements BackedEnum {
//                     ^^^^^^^^^^ CompileError: Non-backed enum Direct cannot implement interface BackedEnum [backed-enum-interface-on-pure-enum]
    case A;
}

enum Indirect implements HasValue {
//                       ^^^^^^^^ CompileError: Non-backed enum Indirect cannot implement interface BackedEnum [backed-enum-interface-on-pure-enum]
    case A;
}
