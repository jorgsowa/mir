===description===
invalidAttributeConstructionWithReturningFunction
===file===
<?php
enum Enumm
{
    case SOME_CASE;
}

#[Attribute]
final class Attr
{
    public function __construct(public Enumm $e) {}
}

final class SomeClass
{
    #[Attr(Enumm::WRONG_CASE)]
    public function anotherMethod(): string
    {
        return "";
    }
}
                
===expect===
UnusedParam@10:32: Parameter $e is never used
