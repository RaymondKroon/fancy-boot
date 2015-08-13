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
    std::vector<form> inner;

    form (form_type type) : type(type) {};
    form (form_type type, const std::string & val) : type(type), value(val) {};

    friend inline std::ostream & operator<<(std::ostream & out, form &v) {
        out << toString(v.type) << " - " << v.value << " - ";
        std::vector<form>::iterator i;
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
            form c(form_type::List);
            std::list<form> inner(read(tokens, form_type::List));
            c.inner.insert(c.inner.end(), inner.begin(), inner.end());

            pop_and_check_front(tokens, end_list);
            result.push_back(c);
            continue;
        }
        else if (token == begin_vector) {
            form c(form_type::Vector);
            std::list<form> inner(read(tokens,form_type::Vector));
            c.inner.insert(c.inner.end(), inner.begin(), inner.end());

            pop_and_check_front(tokens, end_vector);
            result.push_back(c);
            continue;
        }
        else if (token == begin_map) {
            form c(form_type::Map);
            std::list<form> inner(read(tokens,form_type::Vector));
            c.inner.insert(c.inner.end(), inner.begin(), inner.end());

            pop_and_check_front(tokens, end_map);
            result.push_back(c);
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

            form c(form_type::String, value);

            result.push_back(c);
            continue;
        }
        else if (token.at(0) == dispatch) {
            form c(form_type::Dispatch, token);
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

                while (tokens.size() > 0 && tokens.front() != *stop_at) {
                    std::string front(tokens.front());
                    c.inner.push_back(form(form_type::Literal, front));
                    tokens.pop_front();
                }
                if (tokens.front() != *stop_at) {
                    throw std::logic_error("read error: unmatched dispatch closing token: " + token);
                }
                else {
                    tokens.pop_front();
                }
            }

            result.push_back(c);
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

enum class atom_type {Symbol, Number, String, List};

struct atom {
    atom_type type;
    std::string val;
    std::vector<atom> s_expression;
};

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

int main() {
    std::list<std::string> tokenize_test(tokenize("(test,,, 12234dd 2 3) { 1 2} [12, a] #{:a :b} [] #fancy[] (()) #\"regexp\" \"a\""));
    print(tokenize_test);

    std::list<std::string> read_tokens(tokenize("\"3\"(+ 1 2 3)"));
    print(read_tokens);
    std::list<form> form_test(read(read_tokens));
    print(form_test);

    return 0;
}