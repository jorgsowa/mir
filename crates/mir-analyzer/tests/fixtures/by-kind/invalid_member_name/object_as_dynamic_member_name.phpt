===description===
An object is a dynamic property name only when it is `Stringable`, and never a method name.
===file===
<?php
final class Name
{
    public function __toString(): string
    {
        return 'id';
    }
}

final class NotStringable {}

final class Target
{
    public int $id = 1;

    public function id(): int
    {
        return 1;
    }
}

function f(Target $t, Name $name, NotStringable $bad, object $o, string $s): void
{
    echo $t->{$name};
    echo $t->{$bad};
    //        ^^^^ InvalidMemberName: Object of type 'NotStringable' cannot be converted to a property name
    echo $t->{$o};
    echo $t->{$s};
    echo $t->{$name}();
    //        ^^^^^ InvalidMemberName: Method name must be a string, 'Name' given
    echo $t?->{$bad}();
    //         ^^^^ InvalidMemberName: Method name must be a string, 'NotStringable' given
    echo $t->{$s}();
}
