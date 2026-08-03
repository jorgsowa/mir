===description===
M27: a used trait's property with an explicit default value does not need
a constructor — the trait collector previously hardcoded `default: None`
for every trait property regardless of the AST's actual default-value
expression (contrast the class collector, which derives it correctly),
so a class composing only defaulted trait properties still flagged
MissingConstructor.
===file===
<?php
trait TemplateLoader {
    private static array $templates = [];
}

final class Generator {
    use TemplateLoader;
}
===expect===
