<?php

declare(strict_types=1);

/**
 * mir Psalm-plugin host.
 *
 * Long-lived subprocess spawned by mir (crates/mir-plugin/src/psalm/mod.rs).
 * Speaks JSON-lines RPC on stdin/stdout:
 *   -> {"id":1,"method":"init","params":{"projectRoot":"...","plugins":[{"class":"...","configXml":null}]}}
 *   <- {"id":1,"result":{"stubs":[...],"functionIds":[...],"methodClasses":[...],"warnings":[...]}}
 *   -> {"id":2,"method":"functionReturnType","params":{"functionId":"...","argTypes":[...],"snippet":"...","file":"..."}}
 *   <- {"id":2,"result":{"type":"list<int>"}}
 *
 * Psalm plugin classes and Psalm itself are loaded from the analyzed
 * project's own vendor/autoload.php. Shim implementations of Psalm's
 * interfaces are GENERATED at runtime from the installed Psalm's actual
 * interface signatures (see MirShimGenerator), so this host does not break
 * when Psalm adds or changes interface methods.
 */

error_reporting(E_ALL & ~E_DEPRECATED);
ini_set('display_errors', 'stderr');

final class MirShimGenerator
{
    /**
     * Generate and eval a class implementing $interface. $specialBodies maps
     * method name (lowercase) to a PHP body; every other method gets a
     * neutral default derived from its return type. Bodies must use
     * func_get_args() — parameter names differ across Psalm versions.
     */
    public static function implement(string $interface, string $className, array $specialBodies): string
    {
        if (class_exists($className, false)) {
            return $className;
        }
        $rc = new ReflectionClass($interface);
        $methods = [];
        foreach ($rc->getMethods() as $m) {
            if ($m->isStatic()) {
                continue;
            }
            $body = $specialBodies[strtolower($m->getName())]
                ?? self::defaultBody($m->getReturnType(), $m->getName());
            $methods[] = '    public function ' . $m->getName()
                . '(' . self::renderParams($m) . ')'
                . self::renderReturnType($m) . " {\n        $body\n    }";
        }
        $code = 'class ' . $className . ' implements \\' . $interface . " {\n"
            . implode("\n", $methods) . "\n}";
        eval($code);
        return $className;
    }

    private static function renderParams(ReflectionMethod $m): string
    {
        $out = [];
        foreach ($m->getParameters() as $p) {
            $s = '';
            if ($t = $p->getType()) {
                $s .= self::renderType($t) . ' ';
            }
            if ($p->isPassedByReference()) {
                $s .= '&';
            }
            if ($p->isVariadic()) {
                $s .= '...';
            }
            $s .= '$' . $p->getName();
            if ($p->isDefaultValueAvailable()) {
                $s .= ' = ' . var_export($p->getDefaultValue(), true);
            } elseif ($p->allowsNull() && $t && !$p->isVariadic() && !($t instanceof ReflectionNamedType && $t->getName() === 'mixed')) {
                // keep optionalness lax; interfaces may add params later
            }
            $out[] = $s;
        }
        return implode(', ', $out);
    }

    private static function renderReturnType(ReflectionMethod $m): string
    {
        $t = $m->getReturnType();
        return $t === null ? '' : (': ' . self::renderType($t));
    }

    private static function renderType(ReflectionType $t): string
    {
        if ($t instanceof ReflectionUnionType) {
            return implode('|', array_map([self::class, 'renderType'], $t->getTypes()));
        }
        if ($t instanceof ReflectionIntersectionType) {
            return implode('&', array_map([self::class, 'renderType'], $t->getTypes()));
        }
        /** @var ReflectionNamedType $t */
        $name = $t->getName();
        $rendered = $t->isBuiltin() ? $name : '\\' . $name;
        if ($t->allowsNull() && $name !== 'mixed' && $name !== 'null') {
            $rendered = '?' . $rendered;
        }
        return $rendered;
    }

    private static function defaultBody(?ReflectionType $t, string $methodName): string
    {
        if ($t === null) {
            return 'return null;';
        }
        if ($t instanceof ReflectionNamedType) {
            if ($t->allowsNull()) {
                return $t->getName() === 'void' ? 'return;' : 'return null;';
            }
            switch ($t->getName()) {
                case 'void':
                    return 'return;';
                case 'bool':
                    return 'return false;';
                case 'int':
                    return 'return 0;';
                case 'float':
                    return 'return 0.0;';
                case 'string':
                    return "return '';";
                case 'array':
                case 'iterable':
                    return 'return [];';
                case 'mixed':
                    return 'return null;';
                case 'static':
                case 'self':
                    return 'return $this;';
            }
        }
        if ($t instanceof ReflectionUnionType) {
            if ($t->allowsNull()) {
                return 'return null;';
            }
            foreach ($t->getTypes() as $member) {
                if ($member instanceof ReflectionNamedType && $member->isBuiltin()) {
                    return self::defaultBody($member, $methodName);
                }
            }
        }
        return 'throw new \\BadMethodCallException('
            . var_export("$methodName() is not supported by the mir psalm bridge", true) . ');';
    }
}

final class MirPsalmHost
{
    public static ?MirPsalmHost $instance = null;

    /** @var list<string> stub file paths registered by plugins */
    public array $stubs = [];
    /** @var array<string, class-string> lowercase function id -> provider class */
    public array $functionProviders = [];
    /** @var array<string, class-string> lowercase FQCN -> provider class */
    public array $methodProviders = [];
    /** @var list<string> */
    public array $warnings = [];

    public string $currentFile = '';
    public SplObjectStorage $nodeTypes;
    public ?object $nodeTypeProvider = null;
    private ?object $statementsSource = null;
    private $phpParser = null;

    public function __construct()
    {
        $this->nodeTypes = new SplObjectStorage();
        self::$instance = $this;
    }

    // -- RPC dispatch --------------------------------------------------------

    public function dispatch(string $method, array $params): array
    {
        switch ($method) {
            case 'init':
                return $this->init($params);
            case 'functionReturnType':
                return $this->functionReturnType($params);
            case 'methodReturnType':
                return $this->methodReturnType($params);
            case 'shutdown':
                return [];
            default:
                throw new RuntimeException("unknown method: $method");
        }
    }

    private function init(array $params): array
    {
        $root = (string)($params['projectRoot'] ?? getcwd());
        $autoload = $params['autoload'] ?? ($root . '/vendor/autoload.php');
        if (!is_file($autoload)) {
            throw new RuntimeException("no composer autoloader at $autoload — run `composer install` first");
        }
        require $autoload;

        if (!interface_exists('Psalm\\Plugin\\PluginEntryPointInterface')) {
            throw new RuntimeException(
                'Psalm is not installed in this project — psalm plugins need vimeo/psalm '
                . '(it ships the plugin API the plugins are compiled against)'
            );
        }

        $registrationClass = MirShimGenerator::implement(
            'Psalm\\Plugin\\RegistrationInterface',
            'MirRegistrationShim',
            [
                'addstubfile' => '\\MirPsalmHost::$instance->stubs[] = (string)(func_get_args()[0] ?? \'\');',
                'registerpreloadedstubfile' => '\\MirPsalmHost::$instance->stubs[] = (string)(func_get_args()[0] ?? \'\');',
                'registerhooksfromclass' => '\\MirPsalmHost::$instance->registerHooks((string)(func_get_args()[0] ?? \'\'));',
            ]
        );
        $registration = new $registrationClass();

        foreach (($params['plugins'] ?? []) as $spec) {
            $class = (string)($spec['class'] ?? '');
            if ($class === '') {
                continue;
            }
            if (!class_exists($class)) {
                $this->warnings[] = "plugin class $class not found via the project autoloader — skipped";
                continue;
            }
            $config = null;
            if (!empty($spec['configXml'])) {
                try {
                    $config = new SimpleXMLElement((string)$spec['configXml']);
                } catch (Throwable $e) {
                    $this->warnings[] = "plugin $class: invalid config XML: {$e->getMessage()}";
                }
            }
            try {
                $entry = new $class();
                $entry($registration, $config);
            } catch (Throwable $e) {
                $this->warnings[] = "plugin $class entry point failed: {$e->getMessage()} — skipped";
            }
        }

        return [
            'stubs' => array_values(array_unique($this->stubs)),
            'functionIds' => array_keys($this->functionProviders),
            'methodClasses' => array_keys($this->methodProviders),
            'warnings' => $this->warnings,
        ];
    }

    public function registerHooks(string $class): void
    {
        if (!class_exists($class)) {
            $this->warnings[] = "hook class $class not found — skipped";
            return;
        }
        $interfaces = class_implements($class) ?: [];
        $used = false;
        foreach ($interfaces as $iface) {
            if (str_ends_with($iface, 'EventHandler\\FunctionReturnTypeProviderInterface')) {
                foreach ($class::getFunctionIds() as $id) {
                    $this->functionProviders[strtolower(ltrim((string)$id, '\\'))] = $class;
                }
                $used = true;
            } elseif (str_ends_with($iface, 'EventHandler\\MethodReturnTypeProviderInterface')) {
                foreach ($class::getClassLikeNames() as $fqcn) {
                    $this->methodProviders[strtolower(ltrim((string)$fqcn, '\\'))] = $class;
                }
                $used = true;
            } elseif (strpos($iface, 'Psalm\\Plugin\\EventHandler\\') === 0) {
                $short = substr($iface, strrpos($iface, '\\') + 1);
                $this->warnings[] = "$class registers $short — not supported by the mir psalm bridge yet, skipped";
            }
        }
        if (!$used && !$interfaces) {
            $this->warnings[] = "$class implements no recognized Psalm event-handler interfaces";
        }
    }

    // -- Return-type providers ------------------------------------------------

    private function functionReturnType(array $params): array
    {
        $id = strtolower(ltrim((string)($params['functionId'] ?? ''), '\\'));
        $class = $this->functionProviders[$id] ?? null;
        if ($class === null) {
            return ['type' => null];
        }
        try {
            $event = $this->buildEvent(
                'Psalm\\Plugin\\EventHandler\\Event\\FunctionReturnTypeProviderEvent',
                $params,
                ['function_id' => $id]
            );
            $union = $class::getFunctionReturnType($event);
            return ['type' => $union === null ? null : (string)$union];
        } catch (Throwable $e) {
            $this->warnOnce("function provider $class failed for $id: {$e->getMessage()}");
            return ['type' => null];
        }
    }

    private function methodReturnType(array $params): array
    {
        $fqcn = ltrim((string)($params['fqcn'] ?? ''), '\\');
        $method = strtolower((string)($params['methodName'] ?? ''));
        $class = $this->methodProviders[strtolower($fqcn)] ?? null;
        if ($class === null) {
            return ['type' => null];
        }
        try {
            $event = $this->buildEvent(
                'Psalm\\Plugin\\EventHandler\\Event\\MethodReturnTypeProviderEvent',
                $params,
                [
                    'fq_classlike_name' => $fqcn,
                    'method_name_lowercase' => $method,
                    'called_fq_classlike_name' => $fqcn,
                    'called_method_name_lowercase' => $method,
                ]
            );
            $union = $class::getMethodReturnType($event);
            return ['type' => $union === null ? null : (string)$union];
        } catch (Throwable $e) {
            $this->warnOnce("method provider $class failed for $fqcn::$method: {$e->getMessage()}");
            return ['type' => null];
        }
    }

    /**
     * Construct a Psalm event by matching its constructor parameter NAMES
     * against what we can supply — resilient to parameter reordering and
     * additions across Psalm versions.
     */
    private function buildEvent(string $eventClass, array $params, array $extra): object
    {
        if (!class_exists($eventClass)) {
            throw new RuntimeException("$eventClass does not exist in this Psalm version");
        }
        $this->currentFile = (string)($params['file'] ?? 'unknown.php');
        $snippet = isset($params['snippet']) ? (string)$params['snippet'] : null;
        $argTypes = array_map('strval', (array)($params['argTypes'] ?? []));

        $source = $this->statementsSource();
        $callArgs = $this->buildCallArgs($snippet, $argTypes);
        $context = new \Psalm\Context();
        $location = new \Psalm\CodeLocation\Raw(
            (string)$snippet,
            $this->currentFile,
            basename($this->currentFile),
            0,
            max(0, strlen((string)$snippet))
        );

        $byName = $extra + [
            'statements_source' => $source,
            'statements_analyzer' => $source,
            'source' => $source,
            'call_args' => $callArgs,
            'function_args' => $callArgs,
            'context' => $context,
            'code_location' => $location,
            'template_type_parameters' => null,
        ];

        $rc = new ReflectionClass($eventClass);
        $ctor = $rc->getConstructor();
        $args = [];
        foreach ($ctor ? $ctor->getParameters() : [] as $p) {
            if (array_key_exists($p->getName(), $byName)) {
                $args[] = $byName[$p->getName()];
            } elseif ($p->isDefaultValueAvailable()) {
                $args[] = $p->getDefaultValue();
            } elseif ($p->allowsNull()) {
                $args[] = null;
            } else {
                throw new RuntimeException("cannot supply constructor parameter \${$p->getName()} of $eventClass");
            }
        }
        return $rc->newInstanceArgs($args);
    }

    /**
     * Build PhpParser Arg nodes for the call. Primary path re-parses the real
     * call snippet so literal arguments survive; fallback synthesizes
     * variables. Each arg value node gets its mir-inferred type registered in
     * the NodeTypeProvider shim.
     */
    private function buildCallArgs(?string $snippet, array $argTypes): array
    {
        $args = null;
        if ($snippet !== null && $snippet !== '') {
            try {
                $stmts = $this->parser()->parse("<?php\n" . $snippet . ';');
                $finder = new \PhpParser\NodeFinder();
                $call = $finder->findFirst($stmts ?? [], static function ($node): bool {
                    return $node instanceof \PhpParser\Node\Expr\FuncCall
                        || $node instanceof \PhpParser\Node\Expr\MethodCall
                        || $node instanceof \PhpParser\Node\Expr\StaticCall
                        || $node instanceof \PhpParser\Node\Expr\NullsafeMethodCall
                        || $node instanceof \PhpParser\Node\Expr\New_;
                });
                if ($call !== null) {
                    $candidate = $call->getArgs();
                    if (count($candidate) === count($argTypes)) {
                        $args = $candidate;
                    }
                }
            } catch (Throwable $e) {
                // fall through to synthetic args
            }
        }
        if ($args === null) {
            $args = [];
            foreach (array_keys($argTypes) as $i) {
                $args[] = new \PhpParser\Node\Arg(new \PhpParser\Node\Expr\Variable("__mir_arg$i"));
            }
        }

        $this->nodeTypes = new SplObjectStorage();
        foreach ($args as $i => $arg) {
            $typeString = $argTypes[$i] ?? 'mixed';
            try {
                $union = \Psalm\Type::parseString($typeString);
            } catch (Throwable $e) {
                $union = \Psalm\Type::getMixed();
            }
            $this->nodeTypes[$arg->value] = $union;
        }
        return $args;
    }

    private function parser()
    {
        if ($this->phpParser === null) {
            $factory = new \PhpParser\ParserFactory();
            $this->phpParser = method_exists($factory, 'createForHostVersion')
                ? $factory->createForHostVersion()
                : $factory->create(\PhpParser\ParserFactory::PREFER_PHP7);
        }
        return $this->phpParser;
    }

    private function statementsSource(): object
    {
        if ($this->statementsSource === null) {
            $providerClass = MirShimGenerator::implement(
                'Psalm\\NodeTypeProvider',
                'MirNodeTypeProviderShim',
                [
                    'gettype' => '$n = func_get_args()[0] ?? null; $s = \\MirPsalmHost::$instance->nodeTypes;'
                        . ' return ($n !== null && isset($s[$n])) ? $s[$n] : null;',
                    'settype' => '$a = func_get_args(); if (isset($a[0], $a[1])) { \\MirPsalmHost::$instance->nodeTypes[$a[0]] = $a[1]; }',
                ]
            );
            $this->nodeTypeProvider = new $providerClass();

            $sourceClass = MirShimGenerator::implement(
                'Psalm\\StatementsSource',
                'MirStatementsSourceShim',
                [
                    'getnodetypeprovider' => 'return \\MirPsalmHost::$instance->nodeTypeProvider;',
                    'getfilepath' => 'return \\MirPsalmHost::$instance->currentFile;',
                    'getrootfilepath' => 'return \\MirPsalmHost::$instance->currentFile;',
                    'getfilename' => 'return basename(\\MirPsalmHost::$instance->currentFile);',
                    'getrootfilename' => 'return basename(\\MirPsalmHost::$instance->currentFile);',
                    'getaliases' => 'return new \\Psalm\\Aliases();',
                    'getsuppressedissues' => 'return [];',
                    'getsource' => 'return $this;',
                    'gettemplatetypemap' => 'return null;',
                    'setactivephpversion' => 'return;',
                ]
            );
            $this->statementsSource = new $sourceClass();
        }
        return $this->statementsSource;
    }

    private array $seenWarnings = [];

    private function warnOnce(string $message): void
    {
        if (isset($this->seenWarnings[$message])) {
            return;
        }
        $this->seenWarnings[$message] = true;
        fwrite(STDERR, "mir psalm bridge: $message\n");
    }
}

// -- Main loop ----------------------------------------------------------------

$host = new MirPsalmHost();
$stdout = fopen('php://stdout', 'w');

while (($line = fgets(STDIN)) !== false) {
    $line = trim($line);
    if ($line === '') {
        continue;
    }
    $request = json_decode($line, true);
    if (!is_array($request)) {
        continue;
    }
    $id = $request['id'] ?? 0;
    $method = (string)($request['method'] ?? '');

    // Swallow any direct plugin output (echo/print) so it cannot corrupt the
    // JSON protocol on stdout.
    ob_start();
    try {
        $result = $host->dispatch($method, (array)($request['params'] ?? []));
        $response = ['id' => $id, 'result' => $result];
    } catch (Throwable $e) {
        $response = ['id' => $id, 'error' => $e->getMessage()];
    }
    $stray = ob_get_clean();
    if ($stray !== '' && $stray !== false) {
        fwrite(STDERR, $stray);
    }

    fwrite($stdout, json_encode($response, JSON_UNESCAPED_SLASHES) . "\n");
    fflush($stdout);

    if ($method === 'shutdown') {
        break;
    }
}
