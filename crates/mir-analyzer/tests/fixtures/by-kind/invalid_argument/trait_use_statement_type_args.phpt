===description===
`@use Trait<Args>` written on the `use` statement binds the trait's templates like the
class-docblock form; a trait used without type args keeps its templates unbound.
===file===
<?php
/** @template T */
trait BoxTrait {
    /**
     * @param T $value
     * @return T
     */
    public function wrap($value) {
        return $value;
    }
}

final class IntHolder {
    /** @use BoxTrait<int> */
    use BoxTrait;
}

final class StringHolder {
    /** @phpstan-use BoxTrait<string> */
    use BoxTrait;
}

final class PsalmHolder {
    /** @psalm-use BoxTrait<int> */
    use BoxTrait;
}

/** @use BoxTrait<int> */
final class DocHolder {
    use BoxTrait;
}

final class AnyHolder {
    use BoxTrait;
}

trait OuterTrait {
    /** @use BoxTrait<int> */
    use BoxTrait;
}

final class OuterHolder {
    use OuterTrait;
}

enum Suit {
    /** @use BoxTrait<int> */
    use BoxTrait;

    case Hearts;
}

(new OuterHolder())->wrap('x');
//                        ^^^ InvalidArgument: Argument $value of wrap() expects 'int', got '"x"'
Suit::Hearts->wrap('x');
//                 ^^^ InvalidArgument: Argument $value of wrap() expects 'int', got '"x"'

$holder = new IntHolder();
$holder->wrap(1);
$holder->wrap('x');
//            ^^^ InvalidArgument: Argument $value of wrap() expects 'int', got '"x"'

$strings = new StringHolder();
$strings->wrap('x');
$strings->wrap(1);
//             ^ ArgumentTypeCoercion: Argument $value of wrap() expects 'string', got '1' — coercion may fail at runtime

$psalm = new PsalmHolder();
$psalm->wrap(1);
$psalm->wrap('x');
//           ^^^ InvalidArgument: Argument $value of wrap() expects 'int', got '"x"'

$doc = new DocHolder();
$doc->wrap(1);
$doc->wrap('x');
//         ^^^ InvalidArgument: Argument $value of wrap() expects 'int', got '"x"'

$any = new AnyHolder();
$any->wrap(1);
$any->wrap('x');
