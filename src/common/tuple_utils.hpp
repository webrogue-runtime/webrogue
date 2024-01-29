#pragma once
#include <cstddef>
#include <utility>

namespace tupleUtilsImpl {
template <class Tp, std::size_t... J, class F>
static inline F
tupleForEachImpl(Tp &&tp, std::integer_sequence<std::size_t, J...>, F &&f) {
    using A = int[sizeof...(J)];
    return (void)A{((void)f(std::get<J>(std::forward<Tp>(tp))), 0)...},
           std::forward<F>(f);
}

template <class Tp, class F>
static inline F tupleForEachImpl(Tp && /*tp*/,
                                 std::integer_sequence<std::size_t>, F &&f) {
    return std::forward<F>(f);
}

} // namespace tupleUtilsImpl

template <class Tp, class F> F static inline tupleForEach(Tp &&tp, F &&f) {
    using seq = std::make_index_sequence<
        std::tuple_size<typename std::remove_reference<Tp>::type>::value>;
    return tupleUtilsImpl::tupleForEachImpl(std::forward<Tp>(tp), seq(),
                                            std::forward<F>(f));
}

template <typename Function, typename Tuple, size_t... I>
static inline auto call(Function f, Tuple t, std::index_sequence<I...>) {
    return f(std::get<I>(t)...);
}

template <typename Function, typename Tuple>
static inline auto call(Function f, Tuple t) {
    static constexpr auto size = std::tuple_size<Tuple>::value;
    return call(f, t, std::make_index_sequence<size>{});
}
