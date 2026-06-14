===description===
Laravel FP: `method_exists($this, ...)` on a class that inherits `__toString`
emits `ImplicitToStringCast`. The parameter of `method_exists` is `object|string`;
a named object satisfies the `object` arm so no string coercion occurs.
Two paths were broken in `named_object_subtype`. (A) Direct call: bare `TObject`
in the param union was not handled, so any named-object arg failed. Fixed by
returning true when any param atomic is `TObject | TMixed`. (B) After
`method_exists($this, 'x')` narrows `$this` to `TObject`, the join-point type
becomes `TObject | FormRequest`; the bare `TObject` in the arg hit `_ => false`,
causing `all()` to fail. Fixed by adding an explicit `TObject` arg arm that checks
whether the param accepts bare `object`.
===config===
suppress=MissingPropertyType,MissingClosureReturnType,UnusedParam,UnusedVariable,MixedMethodCall,MixedReturnStatement,UndefinedMethod
===file===
<?php

class Request {
    public function __toString(): string { return ''; }
}

class FormRequest extends Request {
    public function validate(): void {
        // After this if/else, $this is narrowed to TObject|FormRequest.
        // Subsequent method_exists calls must not fire ImplicitToStringCast.
        if (method_exists($this, 'validator')) {
            $this->validator();
        }
        if (method_exists($this, 'withValidator')) {
            $this->withValidator();
        }
        if (method_exists($this, 'after')) {
            $this->after();
        }
    }
}
===expect===
