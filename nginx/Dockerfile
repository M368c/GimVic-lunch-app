FROM nginx:alpine
COPY frontend/ /usr/share/nginx/html/
COPY nginx/nginx.conf /etc/nginx/templates/default.conf.template

ENV TZ=Europe/Ljubljana
EXPOSE 80