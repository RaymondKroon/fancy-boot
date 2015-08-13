#include <iostream>
#include <sstream>
#include <string>
#include <vector>
#include <list>
#include <map>
#include <set>

const char _begin_list = '(', _end_list = ')',
            _begin_vector = '[' , _end_vector = ']',
            _begin_map = '{', _end_map ='}',
            _begin_str ='"', _end_str = '"';

const std::string begin_list(1, _begin_list), end_list(1, _end_list),
                  begin_vector(1, _begin_vector), end_vector(1, _end_vector),
                  begin_map(1, _begin_map), end_map(1, _end_map),
                  begin_str(1, _begin_str), end_str(1, _end_str);

const char dispatch = '#';

const char _whitespace[] = {' ', ','};
const char _start_chars[] = {_begin_list, _begin_vector, _begin_map, _begin_str};
const char _end_chars[] = {_end_list, _end_vector, _end_map, _end_str};

const std::set<char> whitespace(_whitespace, _whitespace + sizeof(_whitespace));
const std::set<char> start_chars(_start_chars, _start_chars + sizeof(_start_chars));
const std::set<char> end_chars(_end_chars, _end_chars + sizeof(_end_chars));

// convert given string to list of tokens
std::list<std::string> tokenize(const std::string & str)
{
    std::list<std::string> tokens;
    const char * s = str.c_str();
    while (*s) {
        while (whitespace.find(*s) != whitespace.end()) {
            ++s;
        }

        if (start_chars.find(*s) != start_chars.end() ||
                end_chars.find(*s) != end_chars.end()) {
            const char * t = s;
            ++t;
            tokens.push_back(std::string(s, t));
            s = t;
        }
        else {
            // check if dispatch mode
            bool dispatch_mode = false;
            if (*s == dispatch) {
                dispatch_mode = true;
            }
            const char * t = s;
            while (whitespace.find(*t) == whitespace.end() &&
                    start_chars.find(*t) == start_chars.end() &&
                    end_chars.find(*t) == end_chars.end()) {
                ++t;
            }

            if(dispatch_mode) {
               ++t;
            }

            tokens.push_back(std::string(s, t));
            s = t;
        }
    }

    return tokens;
}

enum class form_type {Literal, String, List, Vector, Map, Dispatch, Outer};

std::string toString(form_type t) {
    switch (t) {
        case form_type::Literal: return "literal";
        case form_type::String: return "string";
        case form_type::List: return "list";
        case form_type::Vector: return "vector";
        case form_type::Map: return "map";
        case form_type::Dispatch: return "dispatch";
        case form_type::Outer: return "outer";
        default:
            return "unknown";
    }
}

struct form {
    form_type type;
    std::string value;
    std::list<form> inner;

    form () {}
    form (form_type type) : type(type) {};
    form (form_type type, const std::string & val) : type(type), value(val) {};

    friend inline std::ostream & operator<<(std::ostream & out, form & v) {
        out << toString(v.type) << " - " << v.value << " - ";
        std::list<form>::iterator i;
        for( i = v.inner.begin(); i != v.inner.end(); ++i) {
            out << "(" << *i << ")";
        }

        return out;
    }
};

void pop_and_check_front(std::list<std::string> &tokens, const std::string &token) {
    std::string front(tokens.front());
    if (front != token) {
        throw std::logic_error("read error: " + token + tokens.front() + " != " + token);
    }

    tokens.pop_front();
}

std::list<form> read (std::list<std::string> & tokens, form_type outer = form_type::Outer) {

    std::list<form> result;
    while (tokens.size() > 0) {

        const std::string token(tokens.front());
        tokens.pop_front();

        if (token == begin_list) {
            form f(form_type::List);
            std::list<form> inner(read(tokens, form_type::List));
            f.inner.insert(f.inner.end(), inner.begin(), inner.end());

            pop_and_check_front(tokens, end_list);
            result.push_back(f);
            continue;
        }
        else if (token == begin_vector) {
            form f(form_type::Vector);
            std::list<form> inner(read(tokens,form_type::Vector));
            f.inner.insert(f.inner.end(), inner.begin(), inner.end());

            pop_and_check_front(tokens, end_vector);
            result.push_back(f);
            continue;
        }
        else if (token == begin_map) {
            form f(form_type::Map);
            std::list<form> inner(read(tokens,form_type::Map));
            f.inner.insert(f.inner.end(), inner.begin(), inner.end());

            pop_and_check_front(tokens, end_map);
            result.push_back(f);
            continue;
        }
        else if (token == begin_str) {

            std::string value;
            while (tokens.size() > 0 && tokens.front() != end_str) {
                value += tokens.front();
                tokens.pop_front();
            }

            if (tokens.front() != end_str) {
                throw std::logic_error("read error: unmatched open string");
            }
            else {
                tokens.pop_front();
            }

            form f(form_type::String, value);

            result.push_back(f);
            continue;
        }
        else if (token.at(0) == dispatch) {

            form dispatch(form_type::Dispatch, token);
            char last = token.back();

            if (start_chars.find(last) != start_chars.end()) {
                const std::string *stop_at;
                switch (last) {
                    case _begin_list:
                        stop_at = &end_list;
                        break;
                    case _begin_vector:
                        stop_at = &end_vector;
                        break;
                    case _begin_map:
                        stop_at = &end_map;
                        break;
                    case _begin_str:
                        stop_at = &end_str;
                        break;
                    default:
                        throw std::logic_error("dispatch read error");
                }

                std::list<std::string> dispatch_inner;

                while (tokens.size() > 0 && tokens.front() != *stop_at) {
                    std::string front(tokens.front());
                    dispatch_inner.push_back(front);
                    tokens.pop_front();
                }

                if (tokens.front() != *stop_at) {
                    throw std::logic_error("read error: unmatched dispatch closing token: " + token);
                }
                else {
                    tokens.pop_front();
                }

                std::list<form> inner(read(dispatch_inner));
                dispatch.inner.insert(dispatch.inner.end(), inner.begin(), inner.end());
            }

            result.push_back(dispatch);

            continue;
        }
        else if (token != end_list && token != end_vector && token != end_map && token != end_str) {
            result.push_back(form(form_type::Literal, token));
            continue;
        }
        else if ((outer != form_type::List && token == end_list) ||
                (outer != form_type::Vector && token == end_vector) ||
                (outer != form_type::Map && token == end_map)) {
            throw std::logic_error("read error: unmatched closing token for " + toString(outer));
        }
        else {
            tokens.push_front(token);
            return result;
        }
    }

    return result;
}

enum class expression_type {Symbol, Number, String, SExpression};

std::string toString(expression_type t) {
    switch (t) {
        case expression_type::Symbol: return "symbol";
        case expression_type::Number: return "number";
        case expression_type::String: return "string";
        case expression_type::SExpression: return "s-expression";
        default:
            return "unknown";
    }
}

struct expression {
    expression_type type;
    std::string value;
    std::vector<expression> s_expression;

    expression (expression_type type) : type(type) {};
    expression (expression_type type, const std::string & val) : type(type), value(val) {};

    friend inline std::ostream & operator<<(std::ostream & out, expression &v) {
        if (v.type != expression_type::SExpression) {
            out << v.value;
        }
        else {
            out << "(";
            std::vector<expression>::iterator i;
            for (i = v.s_expression.begin(); i != v.s_expression.end(); ++i) {
                out << *i << " ";
            }
            out << ")";
        }

        return out;
    }
};

// return true iff given character is '0'..'9'
bool digit(char c) { return std::isdigit(static_cast<unsigned char>(c)) != 0; }

std::list<expression> parse(std::list<form> forms) {
    std::list<expression> result;
    while (forms.size() > 0) {

        form form(forms.front());
        forms.pop_front();

        if (form.type == form_type::List) {

            expression e(expression_type::SExpression);

            std::list<expression> sexp(parse(form.inner));
            e.s_expression.insert(e.s_expression.end(), sexp.begin(), sexp.end());

            result.push_back(e);
            continue;
        }
        else if (form.type == form_type::Vector) {
            expression e(expression_type::SExpression);
            e.s_expression.push_back(expression(expression_type::Symbol, "vector"));

            std::list<expression> sexp(parse(form.inner));
            e.s_expression.insert(e.s_expression.end(), sexp.begin(), sexp.end());

            result.push_back(e);
            continue;
        }
        else if (form.type == form_type::Map) {
            expression e(expression_type::SExpression);
            e.s_expression.push_back(expression(expression_type::Symbol, "hash-map"));

            std::list<expression> sexp(parse(form.inner));
            e.s_expression.insert(e.s_expression.end(), sexp.begin(), sexp.end());

            result.push_back(e);
            continue;
        }
        else if (form.type == form_type::Literal) {

            if (digit(form.value[0]) || (form.value[0] == '-' && digit(form.value[1]))) {
                expression number(expression_type::Number, form.value);
                result.push_back(number);
            }
            else {
                expression symbol(expression_type::Symbol, form.value);
                result.push_back(symbol);
            }

            continue;
        }
        else if (form.type == form_type::String) {

            expression str(expression_type::String, form.value);

            result.push_back(str);
            continue;
        }
        else if (form.type == form_type::Dispatch) {

            if (form.value == "#{" ) {

                expression sexp(expression_type::SExpression);
                expression set(expression_type::Symbol, "hash-set");

                sexp.s_expression.push_back(set);

                //form.inner.pop_front(); // we don't want the map
                std::list<expression> inner(parse(form.inner));
                sexp.s_expression.insert(sexp.s_expression.end(), inner.begin(), inner.end());

                result.push_back(sexp);
            }
            else {
                throw std::logic_error("dispatch unknown: " + form.value);
            }

            continue;
        }
    }

    return result;
}

void print(std::list<std::string> & l) {
    std::list<std::string>::iterator i;
    for( i = l.begin(); i != l.end(); ++i)
        std::cout << *i << ", ";
    std::cout << std::endl;
}

void print(std::list<form> & l) {
    std::list<form>::iterator i;
    for( i = l.begin(); i != l.end(); ++i)
        std::cout << "(" << *i << ")" << std::endl;
}

void print(std::vector<form> & l) {
    std::vector<form>::iterator i;
    for( i = l.begin(); i != l.end(); ++i)
        std::cout << *i;
    std::cout << std::endl;
}

void print(std::list<expression> & l) {
    std::list<expression>::iterator i;
    for( i = l.begin(); i != l.end(); ++i)
        std::cout << *i;
    std::cout << std::endl;
}

int main() {
    std::list<std::string> tokenize_test(tokenize("(test,,, 12234dd 2 3) { 1 2} [12, a] #{:a :b} [] #fancy[] (()) #\"regexp\" \"a\""));
    print(tokenize_test);

    std::list<std::string> read_tokens(tokenize("#{1 2 3}"));
    print(read_tokens);
    std::list<form> form_test(read(read_tokens));
    print(form_test);

    std::list<std::string> parse_tokens(tokenize("[1 2 #{1 2} :a]"));
    print(parse_tokens);
    std::list<form> parse_form(read(parse_tokens));
    print(parse_form);
    std::list<expression> expression_test(parse(parse_form));
    print(expression_test);

    return 0;
}