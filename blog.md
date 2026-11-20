---
layout: default
title: Devlog
permalink: /blog/
description: "Development notes from The Ever-Unfolding Saga: its world, gameplay, systems, art, and ongoing experiments."
---

# Devlog

The Ever-Unfolding Saga is built in the open. These notes follow the work behind the game: design decisions, gameplay systems, world-building, visual development, experiments, and the problems encountered along the way.

TEUS is an active work in progress. Posts distinguish implemented changes from ideas and future plans; a design direction is not a promise that the corresponding feature is already playable.

Subscribe with the <a href="{{ '/feed.xml' | relative_url }}">Atom feed</a>, or [follow development on GitHub](https://github.com/Golden-Bastion/the-ever-unfolding-saga).

{% assign blog_posts = site.posts %}
{% if blog_posts.size > 0 %}
<ul>
{% for post in blog_posts %}
  <li>
    <time datetime="{{ post.date | date_to_xmlschema }}">{{ post.date | date: "%B %-d, %Y" }}</time> — <a href="{{ post.url | relative_url }}">{{ post.title | escape }}</a>
    {% if post.excerpt %}
    <p>{{ post.excerpt | strip_html | normalize_whitespace | truncate: 240 | escape }}</p>
    {% endif %}
  </li>
{% endfor %}
</ul>
{% else %}
<p>No posts yet. New entries will appear here as the project develops.</p>
{% endif %}
