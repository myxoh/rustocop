
        def wrap_routes(routes)
          routes.routes.map { |route| RouteWrapper.new(route) }.reject(&:internal?)
        end

        def load_engines_routes
          engine_routes = @routes.select(&:engine?)

          engines = engine_routes.to_h do |engine_route|
            engine_app_routes = engine_route.rack_app.routes
            engine_app_routes = engine_app_routes.routes if engine_app_routes.is_a?(ActionDispatch::Routing::RouteSet)

            [engine_route.endpoint, wrap_routes(engine_app_routes)]
          end

          engines
        end
