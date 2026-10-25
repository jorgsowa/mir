===description===
Only an enum may implement UnitEnum or BackedEnum, directly or through another interface; an interface may extend them.
===file===
<?php
interface HasLabel extends BackedEnum {}

class Direct implements UnitEnum {
//                      ^^^^^^^^ CompileError: Non-enum class Direct cannot implement interface UnitEnum [enum-interface-on-class]
    public static function cases(): array { return []; }
}

abstract class Backed implements BackedEnum {}
//                               ^^^^^^^^^^ CompileError: Non-enum class Backed cannot implement interface BackedEnum [enum-interface-on-class]

class Indirect implements Countable, HasLabel {
//                                   ^^^^^^^^ CompileError: Non-enum class Indirect cannot implement interface UnitEnum [enum-interface-on-class]
    public function count(): int { return 0; }
}

enum Status: string implements HasLabel {
    case Active = 'active';
}
