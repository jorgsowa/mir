===description===
Variables written before a switch(true) statement where every arm returns must not
be reported as UnusedVariable. The reads inside diverging case bodies (e.g. passed
to `new ClassName(...)`) must propagate out even when all arms diverge.
===config===
suppress=UndefinedClass,MixedArgument,MixedAssignment
===file===
<?php
function parseArgument(string $token): object {
    [$token, $description] = extractDescription($token);

    switch (true) {
        case str_ends_with($token, '?*'):
            return new InputArgument(trim($token, '?*'), 'array', $description);
        case str_ends_with($token, '*'):
            return new InputArgument(trim($token, '*'), 'array_required', $description);
        case str_ends_with($token, '?'):
            return new InputArgument(trim($token, '?'), 'optional', $description);
        default:
            return new InputArgument($token, 'required', $description);
    }
}

function extractDescription(string $token): array {
    return [$token, ''];
}
===expect===
