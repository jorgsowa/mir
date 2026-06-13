===description===
Invalid attribute construction with returning function
===config===
suppress=UnusedParam
===file===
<?php
enum Enumm
{
    case SOME_CASE;
}

#[Attribute]
final class Attr
{
    public function __construct(Enumm $e) {}
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
