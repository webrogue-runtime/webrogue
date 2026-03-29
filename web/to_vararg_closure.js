export function to_vararg_closure(original) {
    return function (...args) {
        return original(args);
    };
}
