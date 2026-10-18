===description===
An `instanceof` check in a callee proves the argument it tests is an object,
so what the callee does on that path only reaches the caller when the
argument passed may hold one. A string or an array of strings from the
request never does; a decoded object, or a value the callee computed from the
argument, still does.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedFunctionCall errorLevel="suppress"/>
    <MixedMethodCall errorLevel="suppress"/>
    <MixedPropertyFetch errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
    <MixedInferredReturnType errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <PossiblyInvalidArgument errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class File {
    public string $name = '';
    public function path(): string { return ''; }
}
class Upload extends File {}

class Storage {
    public function put(string $path, mixed $contents): void {
        if ($contents instanceof File || $contents instanceof Upload) {
            $this->putFile($contents);
            return;
        }
        echo $contents;
    }

    public function putFile(mixed $file): void {
        readfile(is_string($file) ? $file : $file->path());
    }

    public function show(mixed $value): void {
        if (!$value instanceof File) {
            return;
        }
        echo $value->name;
    }

    public function copy(mixed $value): void {
        $copy = $value;
        if ($copy instanceof File) {
            echo $copy->name;
        }
    }

    public function split(mixed $value): void {
        [$first] = [$value, 'fallback'];
        if ($first instanceof File) {
            echo $first->name;
        }
    }

    public function viaHelper(mixed $value): void {
        $same = same($value);
        if ($same instanceof File) {
            echo $same->name;
        }
    }

    public function viaComputation(mixed $value): void {
        $built = wrap($value);
        if ($built instanceof stdClass) {
            echo $built->name;
        }
    }

    /** @param list<mixed> $items */
    public function each(array $items): void {
        foreach ($items as $item) {
            $item = json_decode($item);
            if ($item instanceof stdClass) {
                echo $item->name;
            }
        }
    }
}

class Report {
    private string $title = '';
    private string $header = '';
    private string $footer = '';
    private string $body = '';

    public function section(mixed $value, string $a = '', string $b = ''): void {
        if (func_num_args() > 3 || !$value instanceof File) {
            return;
        }
        echo $value->name . $this->title . $this->header . $this->footer . $this->body;
    }
}

interface Provider {
    /** @param array<string, mixed> $credentials */
    public function retrieve(array $credentials): void;
}

class QueryProvider implements Provider {
    public function retrieve(array $credentials): void {
        $credentials = array_filter($credentials, fn ($key) => $key !== 'password', ARRAY_FILTER_USE_KEY);
        foreach ($credentials as $key => $value) {
            if ($value instanceof Closure) {
                $value($key);
            }
        }
    }
}

class Request {
    /**
     * @psalm-taint-source input
     * @return string|resource
     * @psalm-return ($asResource is true ? resource : string)
     */
    public function getContent(bool $asResource = false) { return ''; }
}

function same(mixed $value) {
    return $value;
}

function wrap(mixed $value) {
    return json_decode($value);
}

function relay(Storage $storage, mixed $value): void {
    $storage->show($value);
}

function unwrap(mixed $value) {
    if ($value instanceof File) {
        return $value->path();
    }
    return 'none';
}

function decode(string $raw): void {
    $decoded = json_decode($raw);
    if ($decoded instanceof stdClass) {
        echo $decoded->name;
    }
}

function redecode(mixed $value): void {
    $value = json_decode($value);
    if ($value instanceof stdClass) {
        echo $value->name;
    }
}

function controller(Storage $storage, Provider $provider, Request $request): void {
    $storage->put('a', $_GET['c']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $storage->put('a', 'static ' . $_GET['c']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $storage->put('a', $request->getContent());
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $storage->show($_GET['c']);
    $storage->copy($_GET['c']);
    $provider->retrieve($_GET);
    $provider->retrieve(['token' => (string) $_GET['token']]);
    echo unwrap($_GET['c']);
    $storage->split($_GET['c']);
    $storage->viaHelper($_GET['c']);
    relay($storage, $_GET['c']);
    (new Report())->section($_GET['c']);
}

function objects(Storage $storage, Provider $provider): void {
    $decoded = json_decode((string) $_GET['j']);
    $storage->put('a', $decoded);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'file'
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $storage->show($decoded);
//  ^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $provider->retrieve(['handler' => $decoded]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs
    echo unwrap($decoded);
//  ^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    relay($storage, $decoded);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function computed(): void {
    decode((string) $_GET['j']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    redecode($_GET['j']);
//  ^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function computedInMethods(Storage $storage): void {
    $storage->viaComputation($_GET['j']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
    $storage->each([$_GET['j']]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
