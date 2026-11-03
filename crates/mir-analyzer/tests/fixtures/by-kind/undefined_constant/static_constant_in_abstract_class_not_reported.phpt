===description===
`static::CONST` in an abstract class may name a constant its subclasses declare; `self::CONST` still fires.
===file===
<?php
abstract class Mapper {
    public function map(): mixed {
        return static::MAP;
    }

    public function own(): mixed {
        return self::MAP;
//             ^^^^^^^^^ UndefinedConstant: Constant Mapper::MAP is not defined
    }
}
