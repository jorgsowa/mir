===description===
A final attribute class used only via #[MyAttr] on an enum declaration or one
of its cases must not be reported UnusedClass.
===config===
suppress=
===file===
<?php
#[Attribute]
final class RouteAttr {
    public function __construct(public string $path = '') {}
}

#[RouteAttr('/status')]
enum Status {
    #[RouteAttr('/status/active')]
    case Active;
}

Status::Active;
===expect===
